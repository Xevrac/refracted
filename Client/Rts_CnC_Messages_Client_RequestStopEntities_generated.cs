using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestStopEntities
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestStopEntities); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestStopEntities)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize array EntityIds
            Rts.Serialization.Reference.Write(s, value.EntityIds, () =>
            {
                s.WriteVarInt32(value.EntityIds.Length);
                for(int i = 0 ; i < value.EntityIds.Length ; ++i)
                {
                    s.Write(value.EntityIds[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestStopEntities)) as Rts.CnC.Messages.Client.RequestStopEntities;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize array EntityIds
            Rts.Serialization.Reference.Read(s, out value.EntityIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
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
