using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_SelectionUpdateChange
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.SelectionUpdateChange); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.SelectionUpdateChange)obj;
            //  Serialize Enable
            s.Write(value.Enable);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.SelectionUpdateChange)) as Rts.CnC.Messages.Client.SelectionUpdateChange;
            //  Deserialize Enable
            s.Read(out value.Enable);

            return value;
        }
        
    }
}
