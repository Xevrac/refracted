using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_FireEvent
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.FireEvent); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.FireEvent)obj;
            //  Serialize EventType
            s.Write(value.EventType);
            //  Serialize EventId
            s.Write(value.EventId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.FireEvent)) as Rts.CnC.Messages.Client.FireEvent;
            //  Deserialize EventType
            s.Read(out value.EventType);
            //  Deserialize EventId
            s.Read(out value.EventId);

            return value;
        }
        
    }
}
