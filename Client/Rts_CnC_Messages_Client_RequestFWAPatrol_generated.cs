using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestFWAPatrol
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestFWAPatrol); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestFWAPatrol)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize AirfieldId
            s.Write(value.AirfieldId);
            //  Serialize array PatrolPoints
            Rts.Serialization.Reference.Write(s, value.PatrolPoints, () =>
            {
                s.WriteVarInt32(value.PatrolPoints.Length);
                for(int i = 0 ; i < value.PatrolPoints.Length ; ++i)
                {
                    s.Write(value.PatrolPoints[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestFWAPatrol)) as Rts.CnC.Messages.Client.RequestFWAPatrol;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize AirfieldId
            s.Read(out value.AirfieldId);
            //  Deserialize array PatrolPoints
            Rts.Serialization.Reference.Read(s, out value.PatrolPoints, () =>
            {
                int length = s.ReadVarInt32();
                SlimMath.Vector3[] tmp = new SlimMath.Vector3[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
