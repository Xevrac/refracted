using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestFWAAttackMove
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestFWAAttackMove); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestFWAAttackMove)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize AirfieldId
            s.Write(value.AirfieldId);
            //  Serialize TargetLocation
            s.Write(value.TargetLocation);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestFWAAttackMove)) as Rts.CnC.Messages.Client.RequestFWAAttackMove;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize AirfieldId
            s.Read(out value.AirfieldId);
            //  Deserialize TargetLocation
            s.Read(out value.TargetLocation);

            return value;
        }
        
    }
}
