using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ShowEntityPlayerPowerTimer
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ShowEntityPlayerPowerTimer); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ShowEntityPlayerPowerTimer)obj;
            //  Serialize Duration
            s.Write(value.Duration);
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ShowEntityPlayerPowerTimer)) as Rts.CnC.Messages.Client.ShowEntityPlayerPowerTimer;
            //  Deserialize Duration
            s.Read(out value.Duration);
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);

            return value;
        }
        
    }
}
